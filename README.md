# simple-contact-form

A simple contact form API

## How to host it

1. Get an API key from resend.com and put it in your .env file

```
RESEND_API_KEY=...
```

2. Create an accounts.json file like this
```
{
    "accounts": [
        {
            "id": "...", // account ID 
            "from_email": "<email-address>", // from email address 
            "email": "<email-address>".      // recepient email address
        }
    ]
}
```

Both the email addresses need to be from domains verified with resend.com

3. Build and run

```
cargo run
```

## How to use it from a client

You can use it from a static HTML page for a form like the following

```html
<form action="<URL>/v1/submit" method="POST">
  <input type="hidden" name="id" value="<ID>" />
  <input type="text" name="name" />
  <input type="text" name="email" />
  <input type="text" name="message" />
  <button>submit</button>  
</form>
```



