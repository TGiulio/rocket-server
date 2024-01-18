# rocket-server

this repository is hopefully the first of a series that I will create as an exercize to learn rust web frameworks.
I started with rocket because it seemed to me to be the simplest and yet complete.

## Application definition

To explore the different frameworks I decided to create applications with the common main features of all or most of the them and I listed the following:

- GET routes which serves static files
- POST routes which receive JSON and/or csv and uses extractors
- a mutable shared state
- request guards
- reading/writing from/to MongoDB

To these, I added some other requisites that are for personal improvement and future use.

- written as much as possible using TDD
- logged with tracer (if needed)
- external script execution (of maybe a simple sum in python)

17/01/2024
I implemented all the features in the most rough way but it was mainly an explorative proof of concept; I did ok with writing test before code but the tests themselves are probably not as comprehensive as they should.
The repository would benefit a lot from a complete refactoring, it will be another excercise.
