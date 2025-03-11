Nerve agent that operate [bettercap](http://bettercap.org/) via its REST API:

Start bettercap with:

```bash
sudo bettercap -eval 'api.rest on'
```

And then run the agent:

```sh
nerve run examples/bettercap-agent --task 'deauth all the apple devices'
```
