# PRODUCE STREAM MESSAGE FROM STRUCT
Use **page-hunter** to produce a stream message from a struct instance.

This example uses [structmap](https://docs.rs/structmap/0.1.6/structmap/) and [structmap-derive](https://docs.rs/structmap-derive/0.1.6/structmap_derive/) crates to convert the struct instance to a string map and then produce the message from it.

To try this example on your local computer, you just need to locate to the respective folder and run the following command:

```bash
	cargo run
```

You need to have a redis server running. Check the host, port, and database in the example main.rs file to determine if you need to change the configuration parameter.

Enjoy it! 😀