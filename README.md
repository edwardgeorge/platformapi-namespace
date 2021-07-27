# `platformapi-namespace`

CLI client for managing "dynamic" namespaces via the Platform API.

## Creating Namespaces

The tool expects the following env variables to be set:

 * `CLIENT_ID`: The client id of the service principal used for the oauth authentication
 * `CLIENT_SECRET`: the client secret of the service principal used for the oauth authentication
 * `SCOPE`: The scope used for the oauth authentication
 * `PLATFORM_API_TENANT`: The tenant used for the outh authentication api call
 * `PLATFORM_API_HOSTNAME`: The hostname of the API endpoint
 * `PLATFORM_API_CLUSTER`: The kubernetes cluster to operator on

Creating a namespace requires a 'productkey' and the namespace name suffix that will be appended to the product key to give the dynamic namespace name.
A ttl can also be provided via the `--ttl` option.

Example:
```
$ platformapi-namespace create --ttl 7d demo-product test
message: Namespace demo-product-test created or updated.
namespace: demo-product-test
expiry: 2021-08-03T09:49:17Z
```

Additionally, options can be provided to specify additional properties of the namespace (such as labels, service principals for vault access etc). see `--help` for more details.
