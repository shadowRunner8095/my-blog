
# Creating a local first isolated dev enviroment


Hi there again! Have you seen the recent news about supply chain attacks, where malicious code is inserted into open source libraries? As we often discuss on this blog, we like to tackle difficult problems and explore creative solutions to help mitigate possible attacks
into your local development enviroment

emulate a malicius pacakge

Information stoler

```typescript
const envs = get_all_ur_envs()
const config_files_maybe_with_tokens = get_files(DEFAULT_FILES)

const tcpServer = awit createTCPServer();

const sender = new Sender(tcpServer);


sender.sendMeTheData({envs, config_files_maybe_with_tokens})
  .retry(10)
  


```

But that is not the only problem see tcp connections are bidirectional
so... avtually I can send u data also and the cleint can interpret that and 
do well something in ur behalf biut that anotehr module

lets create a module taht targets a hipotetical package manager but later

For now stealing data is a problem so.. what can i do?

Easy piece , limit the outgoing internet petitions!,well woudl that not be a 
firewall? and yeah it is a firewall but that means the repsonsability
(whihc is not wrong actually) is in the host machine, to limit sneding data to the 
outrside. 

But lts add a layer that will be the repsonsible for that, meanig well yeah but if I configure.. 
we dont know that and ewach time u google somthign woudl u be whitlisting 
each thing?
