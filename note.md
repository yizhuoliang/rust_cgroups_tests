For write speed test: dd if=/dev/zero of=testfile bs=1G count=1 oflag=dsync
For read speed test (after creating a test file): dd if=testfile of=/dev/null bs=1G