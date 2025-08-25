|                   x                   | sched | browser | persister | requester |
|:-------------------------------------:|:-----:|:-------:|:---------:|:---------:|
|   copart_cmd_lot_search (producer)    |   +   |    -    |     -     |     -     |
|   copart_cmd_lot_search (consumer)    |   -   |    +    |     -     |     -     |
| copart_response_lot_search (producer) |   -   |    +    |     -     |     -     |
| copart_response_lot_search (consumer) |   -   |    -    |     +     |     -     |
|   copart_cmd_lot_images (producer)    |   -   |    -    |     +     |     -     |
|   copart_cmd_lot_images (consumer)    |   -   |    -    |     -     |     +     |
| copart_response_lot_images (producer) |   -   |    -    |     -     |     +     |
| copart_response_lot_images (consumer) |   -   |    -    |     +     |     -     |