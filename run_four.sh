# help with debugging
strace -f -Y -o client4 ./target/debug/bin_client \
   ./bin_client/policies/seccomp_client.json \
   ./tests/nakamoto_config4 \
   ./bin_client/policies/seccomp_nakamoto.json \
   ./tests/_secrets/Wallet.D.json \
   ./bin_client/policies/seccomp_wallet.json \
