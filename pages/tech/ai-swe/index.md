# Improve coding agentes process

## General codign agent flow

Essentially it looks a lot like G Polya's problem solving methodology:

1. User promtps the requiments and context (even some ideas) or instructions
2. Coding agent process your promto together with the source code
3. COdign agent devices a plan
4. Codig anget asks for a ephemral cloud enviroment to run your code
5. Coding agent changes the code
6. Coding agent boots your proejct up
7. Validates if your requeriments are method and nothing was broken in the process
8. If the valadition goes wrong, changes the code readig the errors and goes to seven
9. If vlidation is okay, proposes changes for you to be an arbiter of them
10. You re prompt if somethign is not okay and goes again to 2
11. Of everytin is okay, you proceed to accept the changes

This is a raw simplification, nevertheless I have found some steps
## Potential Bottlenecks by Step

| **Step 1** | **Steps 2 and 3** | **Step 5** | **Step 6** | **Step 7** |
|------------------------------------------------------|---------------------------------------------------------------|--------------------------------------------|----------------------------------------------------------|----------------------------------------------------------|
| Unclear requirements and instructions                | Unclear or hacky code                                         | Scripts poor solutions                     | Tech stack requires many resources                        | Lack of relevant tests                                   |
| Overly guided process                                | Dependencies on contrived solutions                           | Makes code illegible                       | Manual configuration needed                               | Resource-intensive result checking                       |
| Problem too big or wicked                            | Insufficient context                                          |                                            | Additional binaries only in developer environment          | Boots entire app when only a small part is needed        |
| Non-trivial issues                                   | Anemic classes                                                |                                            | Needs access to external resources                        |                                                          |
| Human doesn't understand the real problem            | Unclear domains                                               |                                            |                                                          |                                                          |
|                                                      | Tech stack with limited data                                  |                                            |                                                          |                                                          |


Each possible bottleneck has a potential solution I want to talk about nevetheless, for this article I want to focus on a specific case that has a mix of some of the prevuius problems

## You may not need to boot, build your app

Next.js is somewhat slow in development mode, and in development mode actually
a server is in charge of giving you the page you want to modify.

I think we can render only the small parts we really need

