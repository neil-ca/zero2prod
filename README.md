# zero2prod - Reliable API - Zero Downtime Deployments

## todo 
    [*] write a module to send an email
    [] adapt the logic of our existing POST /subscriptions request handler to match
    the new requirments
    [] write a GET /subscriptions/confirm request handler from scratch

## sketch of how the two handlers should work:
    POST /subscriptions will:
    * add the subscriber details to the database against the subscriber table
    with status equal to pending_confirmation
    * generate a (unique) subscription_token
    * store subscription_token in our database against th e subscriber id in a 
    subscription_token table
    * send an email to the new subscriber containing a link structured as 
    `domain/subscriptions/confirm?token=<subscription_token>`
    * return a 200 Ok

    Once they click on the link, a browser tab will open up and a GET request will be
    fired to our `confirm` endpoint. The request handler will
    * retrieve subscription_token from the query parameters 
    * retrieve the subscription id associated with subscription_token from the 
    subscription_token table
    * update the subscriber status from pending_confirmation to active in the 
    subscriptions table
    * returns a 200 OK

Once an application is serving production traffic, we need to
make sure it is reliable.
Reliable means different things in different contexts. 
If you are selling a data storage solution, for example it should not lose
(or courrupt!) customers data.
In a commercial setting, the definition of reliability for your application will 
often be encoded in a Service Level Agreement (SLA).
An SLA is contractual obligation: you guarantee a certain level of reliability and
commit to compensate your customers (usually with discounts or credits) if your 
service fails to live up to the expectations.
If you are selling access to an API, form example, you will ususally have something 
related to availability - e.g. the API should successfully respond to at least 99.9%
of well-formed incoming requests, often referred to ass "four nines of availability".

There is no silver bullet to build a highly available solution: it requires work
from the application layer all the way down to the infrastructure layer.
One thing is certain, though: if you want to operate a highly available service, 
you should master `zero downtime deployments` - users should be able to use the 
service before, during and after the rollout of a new version of the application
to production.
This is even more important if you are practising continuous deployment: you cannot
release multiple times a day if every release triggers a small outage.

## Deployment Strategies 
### Naive Deployment Before diving deeper into zero downtime deployments 
Let's have a look at the "naive" approach.
Version A of our service is running in production and we want to roll out version B:
* We switch of all instances of version A running the cluster
* We spin up new instances of our application running version B
* We start serving traffic using version B.
There is a non-zero amount of time where there is no application running in the cluster
able to serve user traffic - we are experiencing downtime!
To do better we need to take a closer look at how our infrastructure is set up.
    
    internet --incoming requests--> Load Balancer --> {App, App, App}
## Load Balancers 
Usually supports adding (and removing) backends dynamically. This enables 
a few interesting patterns.
## Horizontal Scaling 
we can add more capacity when experiencing a traffic spike by spinning up more 
replicas of our application (i.e. horizontal scaling).
It helps to spread the load until the work expected of a single instance becomes
manageable.
## Health Checks 
We can ask the load balancer to keep an eye on the health of the registered backends.
Oversimplifyng, health checking can be:
* Passive - the load balancer looks at the distribution of status codes/latency
for each backend to determine if they are healthy or not.
* Active - the load balancer is configured to send a healthy check request to 
each backend on a schedule. If a backend fails to respond with a success status 
code for a long enough time period it is marked as unhealthy and removed.
This is a critical capability to achieve self-healing in a cloud-native environment:
the platform can detect if an application is not behaving as expected and automatically
remove it from the list of available backends to migrate or nullify the impact on users.
## Rolling Update Deployments 
We can leverage our load balancer to perform zero downtime deployments.
Three replicas of version A of our application registered as backends for our 
load balancer.
We want to deploy version B.
We start by spinning up one replica of version B of our application.
When the application is ready to serve traffic (i.e a few health checks requests
have succeeded) we register is as a backend with our load balancer.

A rolling update is not the only possible strategy for a zero downtime deployment 
- `blue-green and canary deployments` are equally popular variations over the same
underlyng principles.

## Database Migrations 
### State is Kept outside the application 
Load balancing relies on a strong assumption: no matter which backend is used 
to serve an incomig request, the outcome will be the same.
To ensure high availability in a fault-prone environment, cloud-native applications
are stateless - they delegate all persistence concerns to external systems (i.e. database).

That's why load balancing works: all backends are talking to the same database
to query and manipulate the same state.
Think of a database as a single gigantic variable. Continuously accessed and 
mutated by all replicas of our application. State is hard.
### Deployments and Migrations 
During a rolling update deployment, the old and the new version of the application
are both serving live traffic, side by side.
From a different prespective: the old and the new version of the application are
using the same database at the same time.

To avoid downtime, we need a database schema that is understood by both versions.
This is not an issue for most of our deployments, but it is a serious constraint 
when we need to evolve the schema.

To move forward with the implementation strategy `confirmation emails`, we need 
to evolve our database schema as follows:
* add a new table, subscription_tokens
* add a new mandatory column, status, to the existing subscriptions table 
