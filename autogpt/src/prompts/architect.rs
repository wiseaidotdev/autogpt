// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

/// Prompt for identifying required architectural scope from a project description.
pub(crate) const ARCHITECT_SCOPE_PROMPT: &str = r#"<role>You are a software architect. Identify which architectural capabilities a project requires.</role>

Return ONLY this JSON object (no markdown, no commentary):
{"crud":bool,"auth":bool,"external":bool}

- "crud": true if the project requires create/read/update/delete operations.
- "auth": true if user authentication (login/logout) is required.
- "external": true if integration with external APIs or data sources is needed.

At least one field must be true.

<project_description>{PROJECT_DESCRIPTION}</project_description>"#;

/// Prompt for selecting relevant public API endpoints for a project.
pub(crate) const ARCHITECT_ENDPOINTS_PROMPT: &str = r#"<role>You are a software architect. Identify public API endpoints that align with the project behaviors.</role>

<rules>
- Prioritize endpoints that do not require API keys.
- Return a JSON array of URL strings: ["url_1", "url_2", ...]
- Output only the JSON array. No commentary.
</rules>

<project_description>{PROJECT_DESCRIPTION}</project_description>"#;

/// Prompt for generating Python architecture diagrams using the diagrams library.
pub(crate) const ARCHITECT_DIAGRAM_PROMPT: &str = r#"<role>You are a software architect. Generate Python code using the `diagrams` library to visualize the described architecture.</role>

<rules>
- Generate accurate, runnable Python code using the `diagrams` library.
- Adapt the examples below to match the specific requirements.
- Output only the Python source code. No backticks, no commentary.
</rules>

<examples>
<example name="Stateful Architecture on Kubernetes">
from diagrams import Cluster, Diagram
from diagrams.k8s.compute import Pod, StatefulSet
from diagrams.k8s.network import Service
from diagrams.k8s.storage import PV, PVC, StorageClass

with Diagram("Stateful Architecture", show=False):
    with Cluster("Apps"):
        svc = Service("svc")
        sts = StatefulSet("sts")
        apps = []
        for _ in range(3):
            pod = Pod("pod")
            pvc = PVC("pvc")
            pod - sts - pvc
            apps.append(svc >> pod >> pvc)
    apps << PV("pv") << StorageClass("sc")
</example>

<example name="Exposed Pod with 3 Replicas on Kubernetes">
from diagrams import Diagram
from diagrams.k8s.clusterconfig import HPA
from diagrams.k8s.compute import Deployment, Pod, ReplicaSet
from diagrams.k8s.network import Ingress, Service

with Diagram("Exposed Pod with 3 Replicas", show=False):
    net = Ingress("domain.com") >> Service("svc")
    net >> [Pod("pod1"), Pod("pod2"), Pod("pod3")] << ReplicaSet("rs") << Deployment("dp") << HPA("hpa")
</example>
</examples>

<user_request>{USER_REQUEST}</user_request>"#;
