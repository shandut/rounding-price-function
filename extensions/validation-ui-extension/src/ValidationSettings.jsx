import React, { useState } from "react";
import {
  reactExtension,
  useApi,
  Text,
  Box,
  FunctionSettings,
  Section,
  NumberField,
  BlockStack,
  Banner,
  InlineStack,
} from "@shopify/ui-extensions-react/admin";

const TARGET = "admin.settings.validation.render";
const METAFIELD_NAMESPACE = "$app:cart-validation";
const METAFIELD_KEY = "minimum-order-threshold";

export default reactExtension(TARGET, async (api) => {
  console.log("Extension context:", api.data);
  
  // First, get the current metafield value
  const currentValue = await getCurrentMetafieldValue(api.query);
  console.log("Current metafield value from query:", currentValue);

  const existingDefinition = await getMetafieldDefinition(api.query);
  if (!existingDefinition) {
    // Create a metafield definition for persistence if no pre-existing definition exists
    const metafieldDefinition = await createMetafieldDefinition(api.query);

    if (!metafieldDefinition) {
      throw new Error("Failed to create metafield definition");
    }
  }

  // Use the queried value or fall back to the default
  const initialThreshold = currentValue ? parseFloat(currentValue) : 1000;
  console.log("Initial threshold:", initialThreshold);

  return <ValidationSettings initialThreshold={initialThreshold} />;
});

// Function to get the current metafield value
async function getCurrentMetafieldValue(adminApiQuery) {
  const query = `#graphql
    query GetMetafieldValue {
      metafieldDefinitions(first: 1, ownerType: VALIDATION, namespace: "${METAFIELD_NAMESPACE}", key: "${METAFIELD_KEY}") {
        nodes {
          id
          metafields(first: 1) {
            nodes {
              value
            }
          }
        }
      }
    }
  `;

  const result = await adminApiQuery(query);
  console.log("Get current metafield value result:", result);
  
  const value = result?.data?.metafieldDefinitions?.nodes[0]?.metafields?.nodes[0]?.value;
  
  // Add more detailed logging
  if (value) {
    console.log("Found saved threshold value:", value);
  } else {
    console.log("No saved threshold value found, will use default");
  }
  
  return value;
}

function ValidationSettings({ initialThreshold }) {
  const [errors, setErrors] = useState([]);
  // State to keep track of minimum order threshold setting
  const [threshold, setThreshold] = useState(initialThreshold);

  console.log("Current threshold state:", threshold);

  const { applyMetafieldChange } = useApi(TARGET);

  const onError = (error) => {
    console.error("Error occurred:", error);
    setErrors([error]);
  };

  const onChange = async (value) => {
    console.log("Threshold changed to:", value);
    setErrors([]);
    setThreshold(value);

    // Store the value directly as a string
    const result = await applyMetafieldChange({
      type: "updateMetafield",
      namespace: METAFIELD_NAMESPACE,
      key: METAFIELD_KEY,
      value: String(value), // Store as string directly
    });

    console.log("Metafield update result:", result);

    if (result.type === "error") {
      setErrors([result.message]);
    }
  };

  return (
    // Note: FunctionSettings must be rendered for the host to receive metafield updates
    <FunctionSettings onError={onError}>
      <ErrorBanner errors={errors} />
      <Section heading="Minimum Order Threshold Settings">
        <BlockStack paddingBlock="large">
          <InlineStack>
            <Box minInlineSize="50%">
              <Text>Set the minimum order total required to proceed with checkout</Text>
            </Box>
            <Box minInlineSize="50%">
              <NumberField
                value={threshold}
                min={0}
                step={0.01}
                label="Minimum Order Total ($)"
                onChange={(value) => onChange(Number(value))}
              />
            </Box>
          </InlineStack>
        </BlockStack>
      </Section>
    </FunctionSettings>
  );
}

function ErrorBanner({ errors }) {
  if (errors.length === 0) return null;

  return (
    <Box paddingBlockEnd="large">
      {errors.map((error, i) => (
        <Banner key={i} title="Errors were encountered" tone="critical">
          {error}
        </Banner>
      ))}
    </Box>
  );
}

async function getMetafieldDefinition(adminApiQuery) {
  const query = `#graphql
    query GetMetafieldDefinition {
      metafieldDefinitions(first: 1, ownerType: VALIDATION, namespace: "${METAFIELD_NAMESPACE}", key: "${METAFIELD_KEY}") {
        nodes {
          id
        }
      }
    }
  `;

  const result = await adminApiQuery(query);
  console.log("Metafield definition query result:", result);

  return result?.data?.metafieldDefinitions?.nodes[0];
}

async function createMetafieldDefinition(adminApiQuery) {
  const definition = {
    access: {
      admin: "MERCHANT_READ_WRITE",
    },
    key: METAFIELD_KEY,
    name: "Minimum Order Threshold Configuration",
    namespace: METAFIELD_NAMESPACE,
    ownerType: "VALIDATION",
    type: "single_line_text_field", // Using single_line_text_field for direct string storage
  };

  const query = `#graphql
    mutation CreateMetafieldDefinition($definition: MetafieldDefinitionInput!) {
      metafieldDefinitionCreate(definition: $definition) {
        createdDefinition {
          id
        }
      }
    }
  `;

  const variables = { definition };
  const result = await adminApiQuery(query, { variables });
  console.log("Create metafield definition result:", result);

  return result?.data?.metafieldDefinitionCreate?.createdDefinition;
}