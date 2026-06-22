# Build LLVM

1. Download [llvm-project](https://github.com/llvm/llvm-project)
2. Install `CMake`/`Ninja`
3. Build tool, such as `llvm-dis`
    ``` shell
    mkdir build
    cd build
    cmake -G Ninja ../llvm-project-llvmorg-version/llvm -DCMAKE_BUILD_TYPE=Debug
    ninja llvm-dis
    bin/llvm-dis -debug file.bc -o file.ll
    ```
