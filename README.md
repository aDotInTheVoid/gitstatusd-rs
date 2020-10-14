
    git clone --recurse-submodules
    cd gitstatusd
    ./build -w
    cargo test

    echo -nE id$'\x1f'`pwd`$'\x1e' | ./gitstatusd/usrbin/gitstatusd | bat -A