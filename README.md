# zero2prod - Reliable API - Zero Downtime Deployments

## todo 
    [] As the blog author, I want to send an email to all my confirmed subscribers
    [] Fix test related to deliver Email's and idempotency
        1 - newsletter_creation_is_idempotent
        2 - concurrent_form_submission_is_handled_gracefully
        3 - newsletters_are_delivered_to_confirmed_subscribers
    [] Document the API
    [] Use my own SMTP server
    [] deploy in k3s

## Limitations of the Naive approach
1. Security
Our POST /newsletters endpoint in unprotected - anyone can fire a request to it 
and broadcast to our entire audience
2. You only get one shot
As soon you hit POST /newsletter, your content goes out to your entire mailing list.
No chance to edit or review it in draft mode before giving the green light for publising.
3. Performance 
We are sending emails out one at a time.
Latency is going to be horrible for newsletters with a sizeable audience.
4. Fault tolerance 
If we fail to dispatch one email we bubble up the error using ? and return a 500
to the caller. The remaining emails are never sent, nor we retry to dispatch the failed one.

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
### Multi-step Migrations 
A big bang release won't cut it - we need to get there in multiple, smaller steps.
The pattern is somewhat similar to what we see in test-driven development: we
don't change code and test at the same time - one of the two needs to stay still
while the other changes.

## Authentication
### Error Messages Must Be Ephimeral 
For now the error is rendered as expected and nobody can tamper 
with our messages thanks to the HMAC tag. 

Don't abuse of query params to despite of being easy to pass them along in the 
value of the `Location` header when redirecting back to the login form on failures.
URLs are stored in the browser history; dirty and annoying.
Cookies are a great alternative, we can use them to implement the same strategy
* The user enters invalid credentials and submits the form;
* POST /login sets a cookie containing the error message and redirects the user back
to GET /login
* GET /login's request handler checks the cookies to see if there is an error message
to be rendered
* GET /login returns the HTML form to the caller and deletes the error message from the cookie.

## Integrations tests for login failures

## Fault-tolerant Workflows 
Errors can happen in network I/O, Databases, Extern API's Errors.
For API's calls that we can't observe if a request has been sent to the server 
once or multiple times is idempotent.
The caller generates a unique identifier, the idempotency key for every state-altering
operation they want to perform.

### Idempotency
Ensures that a particular action or request has the same effect regardless of how many
times it is executed. 
1. Use appropiate HTTP Methods, POST is which is not.
2. Design Stateless Operations: Your endpoints should not rely on the current state
of the server.
3. Generate unique identifiers

### Backward recovery and Forward
- Backward recovery tries to achieve a semantic rollback by executing compensating actions.
Is not good fi for our newsletter delivery system - we cannot "unsend" an email nor
would it make sense to send a follow-up email asking subscribers to ignore the email
we sent before (it,d be funny though).

We must try to perform forward recovery - drive the overall workfow to completion
even if one or more sub-tasks did not succeed.

- Forward recovery
has 2 types active and passive: pushes on the API caller the responsability to drive 
the workflow to completion. The request handler leverages checkpoints to kkep track
of its progress - e.g. "123 email have been sent out". If the handler crashes, the next
API call will resume processing from the latest enpoint.

Active recovery, instead, does not require the caller to do anything apart from kicking
off the workflow. The system must self-heal.
We would rely on a background process - e.g. a background task on our API - to detect
newsletter issues whose delivery stopped halfway. The process would then drive the delivery
to completion.
Healing would happen asynchronously - outside the lifecycle of the original POST
/admin/newsletters request.

### Asynchronous Processng
We do not want the author to receive an error back from the API while, under the hood,
newsletter deivery has been kicked off.
We can improve the user experience by changing the expectations for `POST /admin/newsletters`.
We can reduce its scope: a successful form submission will mean that the newsletter
has been validated and will be delivered to all subscriers, asynchronously.

The request handlers is no longer going to dispatch emails -it will siply enqueue
a list of tasks that will be fulfilled asynchronously by a set of background workers.
At a glance, it might look like a small difference - we are just shifting around
when work needs to happen. But it has a powerful implication. we recover transactionality.
Our subscriber's data, our idempotency records, the task queue can be wrapped in 
a transaction.
