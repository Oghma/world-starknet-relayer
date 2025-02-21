#!/usr/bin/env bash
set -e

# Function to update or add environment variable
update_env_var() {
    local var_name=$1
    local var_value=$2

    if grep -q "^$var_name=" "$ENV_FILE"; then
        echo -e "${BLUE}$var_name already exists, replacing in $ENV_FILE...${NC}"
        # macOS compatibility: BSD sed requires extension for -i, empty string for no backup
        # Linux compatibility: GNU sed ignores empty extension while maintaining same syntax
        sed -i "" "s|^$var_name=.*|$var_name=\"$var_value\"|" "$ENV_FILE"
    else
        echo -e "${BLUE}Appending $var_name to $ENV_FILE...${NC}"
        echo "$var_name=\"$var_value\"" >> "$ENV_FILE"
    fi
}

ORIGINAL_DIR="$(pwd)"
STARKNET_DIR="$ORIGINAL_DIR/contracts"

# Load .env file if exists
if [ -f "$ORIGINAL_DIR/.env" ]; then
    echo -e "${BLUE}Sourcing .env file${NC}"
    set -a
    source "$ORIGINAL_DIR/.env"
    set +a
fi

# Define colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color
BOLD='\033[1m'
RED='\033[0;31m'

# Check required environment variables
if [ -z "$STARKNET_ACCOUNT" ]; then
    echo -e "\n${RED}Error: STARKNET_ACCOUNT environment variable is not set${NC}"
    exit 1
fi

if [ -z "$STARKNET_KEYSTORE" ]; then
    echo -e "\n${RED}Error: STARKNET_KEYSTORE environment variable is not set${NC}"
    exit 1
fi

if [ -z "$STARKNET_RPC" ]; then
    echo -e "\n${RED}Error: STARKNET_RPC environment variable is not set${NC}"
    exit 1
fi

# Now deploy Starknet contracts
cd "$STARKNET_DIR"

echo -e "\n${BLUE}${BOLD}Deploying Starknet contracts...${NC}"

# Relayer Store contract
echo -e "\n${YELLOW}Declaring Relayer Store contract...${NC}"
RELAYER_STORE_HASH=$(starkli declare --compiler-version 2.9.1 ./target/dev/world_relayer_store_WorldRelayerStore.contract_class.json \
    --strk -w | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
echo -e "${GREEN}Class hash declared: ${BOLD}$RELAYER_STORE_HASH${NC}\n"

echo -e "${YELLOW}Deploying Relayer Store contract...${NC}"
RELAYER_STORE_ADDRESS=$(starkli deploy $RELAYER_STORE_HASH \
    --strk -w | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
echo -e "${GREEN}Contract address: ${BOLD}$RELAYER_STORE_ADDRESS${NC}\n"

# Universal ECIP contract
echo -e "${YELLOW}Declaring Universal ECIP contract...${NC}"
ECIP_HASH=$(starkli declare --compiler-version 2.9.1 ./target/dev/verifier_UniversalECIP.contract_class.json \
    --strk -w | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
echo -e "${GREEN}Class hash declared: ${BOLD}$ECIP_HASH${NC}\n"

# Groth16 Verifier contract
echo -e "${YELLOW}Declaring Groth16 Verifier contract...${NC}"
VERIFIER_HASH=$(starkli declare --compiler-version 2.9.1 ./target/dev/verifier_Risc0Groth16VerifierBN254.contract_class.json \
    --strk -w | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
echo -e "${GREEN}Class hash declared: ${BOLD}$VERIFIER_HASH${NC}\n"

echo -e "${YELLOW}Deploying Groth16 Verifier contract...${NC}"
VERIFIER_ADDRESS=$(starkli deploy $VERIFIER_HASH $ECIP_HASH \
    --strk -w | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
echo -e "${GREEN}Contract deployed at: ${BOLD}$VERIFIER_ADDRESS${NC}\n"

echo -e "${YELLOW}Declaring World Relayer Verifier contract...${NC}"
WORLD_RELAYER_VERIFIER_HASH=$(starkli declare --compiler-version 2.9.1 ./target/dev/verifier_WorldRelayerVerifier.contract_class.json \
    --strk -w | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
echo -e "${GREEN}Class hash declared: ${BOLD}$WORLD_RELAYER_VERIFIER_HASH${NC}\n"

echo -e "${YELLOW}Deploying World Relayer Verifier contract...${NC}"
WORLD_RELAYER_VERIFIER_ADDRESS=$(starkli deploy $WORLD_RELAYER_VERIFIER_HASH $VERIFIER_ADDRESS $RELAYER_STORE_ADDRESS \
    --strk -w | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
echo -e "${GREEN}Contract deployed at: ${BOLD}$WORLD_RELAYER_VERIFIER_ADDRESS${NC}\n"

echo -e "${YELLOW}Initializing Relayer Store contract...${NC}"
starkli invoke $RELAYER_STORE_ADDRESS initialize $VERIFIER_ADDRESS \
    --strk -w
echo -e "${GREEN}Relayer Store contract initialized${NC}\n"

# Store deployed addresses in .env if exists
if [ -f "$ORIGINAL_DIR/.env" ]; then
    ENV_FILE="$ORIGINAL_DIR/.env"
    
    echo -e "${BLUE}Storing contract addresses in .env${NC}"
    update_env_var "RELAYER_STORE_ADDRESS" "$RELAYER_STORE_ADDRESS"
    update_env_var "VERIFIER_ADDRESS" "$VERIFIER_ADDRESS"
    update_env_var "WORLD_RELAYER_VERIFIER_ADDRESS" "$WORLD_RELAYER_VERIFIER_ADDRESS"
fi

echo -e "\n${GREEN}${BOLD}All contracts deployed!${NC}"
