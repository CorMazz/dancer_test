#!/bin/sh

set -e  # Exit on error

generate_keypair() {
  name_prefix="$1"

  # Step 1: Generate RSA private key
  openssl genpkey -algorithm RSA -out "${name_prefix}_private.pem" -pkeyopt rsa_keygen_bits:2048

  # Step 2: Extract the public key
  openssl rsa -pubout -in "${name_prefix}_private.pem" -out "${name_prefix}_public.pem"

  # Step 3: Base64 encode (platform-aware)
  if base64 --help 2>/dev/null | grep -q -- "-w"; then
      PRIVATE_B64=$(base64 -w 0 "${name_prefix}_private.pem")
      PUBLIC_B64=$(base64 -w 0 "${name_prefix}_public.pem")
  else
      PRIVATE_B64=$(base64 "${name_prefix}_private.pem" | tr -d '\n')
      PUBLIC_B64=$(base64 "${name_prefix}_public.pem" | tr -d '\n')
  fi

  # Step 4: Export as environment variables
  export "${name_prefix}_PRIVATE_KEY=$PRIVATE_B64"
  export "${name_prefix}_PUBLIC_KEY=$PUBLIC_B64"

  # Step 5: Append to .env_keys file
  echo "${name_prefix}_PRIVATE_KEY=\"$PRIVATE_B64\"" >> .env_keys
  echo "${name_prefix}_PUBLIC_KEY=\"$PUBLIC_B64\"" >> .env_keys

  # Step 6: Clean up
  rm "${name_prefix}_private.pem" "${name_prefix}_public.pem"

  # Confirmation
  echo "${name_prefix}_PRIVATE_KEY (truncated): $(echo "$PRIVATE_B64" | cut -c1-60)..."
  echo "${name_prefix}_PUBLIC_KEY (truncated):  $(echo "$PUBLIC_B64" | cut -c1-60)..."
}

# Clear existing .env_keys
> .env_keys

# Generate both key pairs
echo "🔐 Generating access token key pair..."
generate_keypair "ACCESS_TOKEN"

echo "🔐 Generating refresh token key pair..."
generate_keypair "REFRESH_TOKEN"

echo "✅ All keys generated and saved to .env_keys"
