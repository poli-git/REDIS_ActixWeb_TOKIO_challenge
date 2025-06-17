# Async Worker Process

Instructions and other relevant references for deploying and maintaining async worker service.

## Related Services

### Dependencies

The async worker depends on and interacts directly with the following:

-   Redis Cache
-   Postgresql DB

## Initial Rollout

If the webapp platform is being deployed, at least one async worker must also be deployed in the same environment. You can deploy multiple workers simultaneously to increase processing throughput.

If there are no async workers running, any requests to the webapp platform will only return or operate on existing records stored in the Redis Cache. Since no new data or updates will be retrieved from providers, there is a risk of serving outdated events.

#### Steps

Find here detailed steps:
-   Ensure that hosted versions of [dependencies](#dependencies) are running.
-   Prior to deployment, review the [ENV vars section of the README.md](README.md#env-vars), paying special attention to the f
-   On a given environment, deploy the desired number of new async workers
-   Configure corresponding providers to the Providers table from the DB as listed below:
  
    ```shell
        INSERT INTO providers 
        (
            name,
            description,
            url,
            is_active
        )
        VALUES 
        (   
            'FeverUp',
            'This is the description for FeverUp',
            'https://provider.code-challenge.feverup.com/api/events',
            TRUE
        );
    ```