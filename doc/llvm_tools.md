# LLVM Tools

## 1. `llvm-dis`: `.bc` to `.ll`

- Check `.bc`

    ``` bash
    llvm-dis -disable-output file.bc
    ```

- `.bc` to `.ll`

    ``` bash
    llvm-dis file.bc -o file.ll
    ```

## 2. `llvm-as`: `.ll` to `.bc`

- Check `.ll`

    ``` bash
    llvm-as -disable-output file.ll
    ```

- `.ll` to `.bc`

    ``` bash
    llvm-as file.ll -o file.bc

    # -o file.bc
    llvm-as file.ll
    ```

## 3. `llvm-bcanalyzer`: display `.bc`

``` bash
llvm-bcanalyzer --dump file.bc > file.txt
```
