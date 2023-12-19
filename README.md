REST api I made that connects to my smart home system (interra technology, uses KNX) and controls my bedroom lights and ac. 

i gave the endpoints to my friend who proceeded to make the goofiest and most immersive game ever. 10/10
### environment variables
```
AUTH_TOKEN: key necessary to include in auth header to use api
TCP_IP: ip address of smart home network to connect to
PORT: port of tcp server to connect to
USERNAME: username of tcp client
PASSWORD: password of tcp client
```
when you run it, go to the root endpoint for docs 👍

**note for any normal people reading this: while I am decently proud of the idea, this entire project is a joke. please excuse any
humor you see in api responses. in the future, I may repurpose this and use it with a TRMNL or something!**

(code is from 2023, so please excuse any bad practices or outdated libraries)