# Uninstalling Context Kernel (macOS)

This package installs files under `/usr/local`. To remove them:

```sh
sudo rm /usr/local/bin/context-kernal /usr/local/bin/context-kernel
sudo rm -r /usr/local/share/doc/context-kernel
sudo pkgutil --forget com.godspeedai.context-kernel
```

Your data is kept. Databases and artifacts live in
`~/Library/Application Support/Context Kernel`, which these steps never touch.
Delete that directory yourself only if you want to discard your data.
