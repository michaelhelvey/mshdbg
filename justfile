run *args:
    cargo build
    install_name_tool -add_rpath /Applications/Xcode.app/Contents/SharedFrameworks ./target/debug/mshdbg
    ./target/debug/mshdbg {{args}}
