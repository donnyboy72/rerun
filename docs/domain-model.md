# Initial Domain Model

This conceptual model describes the domain of recording and exploring time-based, multimodal observations.
It represents the problem domain rather than Rerun's source-code structure.

```mermaid
classDiagram
    class Analyst {
        role
    }

    class Recording {
        name
        creationTime
        description
    }

    class DataSource {
        name
        sourceKind
    }

    class ObservedSubject {
        name
        identifier
    }

    class Observation {
        timestamp
        value
        modality
    }

    class Timeline {
        name
        timeBasis
    }

    class View {
        name
        presentationKind
    }

    Analyst "0..*" -- "0..*" Recording : explores
    Recording "1" *-- "0..*" Observation : contains
    Recording "1" *-- "1..*" Timeline : organizes data on
    Recording "1" -- "0..*" View : is presented through
    DataSource "1" -- "0..*" Observation : produces
    ObservedSubject "1" -- "0..*" Observation : is described by
    Observation "0..*" -- "1..*" Timeline : occurs on
    View "0..*" -- "1..*" ObservedSubject : displays
```

The model treats an observation as a recorded fact about one observed subject from one data source.
A recording groups observations on one or more timelines, and analysts explore those observations through views.
