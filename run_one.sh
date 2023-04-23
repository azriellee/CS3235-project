# help with debugging
python3 ./tests/_bots/botA-1.py > botpipeA &
strace -f -Y -o client1 ./target/debug/bin_client \
   ./bin_client/policies/seccomp_client.json \
   ./tests/nakamoto_config1 \
   ./bin_client/policies/seccomp_nakamoto.json \
   ./tests/_secrets/Wallet.A.json \
   ./bin_client/policies/seccomp_wallet.json \
   botpipeA \