# help with debugging
strace -f -Y -o client2 ./target/debug/bin_client \
   ./bin_client/policies/seccomp_client.json \
   ./tests/nakamoto_config2 \
   ./bin_client/policies/seccomp_nakamoto.json \
   ./tests/_secrets/Wallet.B.json \
   ./bin_client/policies/seccomp_wallet.json \
