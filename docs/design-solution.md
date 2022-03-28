```mermaid
sequenceDiagram
    autonumber
    loop each symbol
        main ->>+ Downloader: new
        Downloader ->>- Processor: new
        main ->>+ Downloader: start
        Downloader ->>- Processor: start
    end
    loop interval FetchMessage
        Downloader ->> Downloader: fetch data
        Downloader ->> Processor: ProcessMessage
        Processor ->> Processor: calculate and output
    end
```