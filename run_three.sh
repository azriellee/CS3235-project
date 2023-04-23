# help with debugging
strace -f -Y -o client3 ./target/debug/bin_client \
   ./bin_client/policies/seccomp_client.json \
   ./tests/nakamoto_config3 \
   ./bin_client/policies/seccomp_nakamoto.json \
   ./tests/_secrets/Wallet.C.json \
   ./bin_client/policies/seccomp_wallet.json \
