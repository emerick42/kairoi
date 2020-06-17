# Kairoi Client Protocol

## Quick Words

Kairoi clients communicate with Kairoi servers using a protocol called KCP (Kairoi Client Protocol). KCP is a simple request-response UTF-8 encoded text-based protocol over the TCP protocol. It's usually used on the port 5678.

Here is a representative example of a communication between a client and a server, using Kairoi instructions (read more on available instructions in the [Kairoi Instructions documentation](instructions.md)) :

```
Client: SET app.domain.example_job.0 "2020-05-26 22:26:18"\n
Server: OK\n
Client: "UNSET" "app domain ex\\ampl\"e_job 0"\n
Server: OK\n
```

## Usage

### Request-Response

KCP is a request-response based protocol. The client CAN send a request, then MUST wait for the server to send a response, before being able to send another request. The server MUST wait for a client to send a request before doing anything.

### Message

The entire content of a request (or a response) is called a message. A message is a sequence of arguments (at least one), separated by any number of spaces (` `), and terminated by the line feed (`\n` or `\U+000A`) character.

### Argument

Arguments are represented by the only data type defined by this protocol: strings.

### String

A string is a valid sequence of any UTF-8 encoded characters. It can be written in two forms: the simple form, and the universal form.

#### Simple String

Simple strings allow any UTF-8 encoded characters BUT spaces (` `), double-quotes (`"`) and line feeds (`\n` or `\U+000A`).

Here are examples of valid simple strings (one simple string per line):

```
Hello
app.domain$/!#*,toto
IUseEmojis\U+1F631Haha
```

#### Universal String

Universal strings allow usage of literally any UTF-8 encoded characters, but they MUST be surrounded by double-quotes (`"`). It uses backslash (`\`) as the escaping character. Only backslashes (`\`) and double-quotes (`"`) MUST be escaped.

Here are exemples of valid universal strings (one universal string per line):

```
"Hello"
"Hello, world!"
"I can\"con$tain\U+1F631everythi\U+000Ag\\."
```

## Internals
