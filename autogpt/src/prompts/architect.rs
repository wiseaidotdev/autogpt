// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

/// Prompt for identifying required architectural scope from a project description.
pub(crate) const ARCHITECT_SCOPE_PROMPT: &str = r#"<role>You are a master software architect. Identify which architectural capabilities a project requires based on its description.</role>

<rules>
- Return ONLY a valid JSON object: `{"crud": true, "auth": false, "external": true}`.
- "crud": true if the project requires create/read/update/delete operations.
- "auth": true if user authentication (login/logout) is required.
- "external": true if integration with external APIs or data sources is needed.
- At least one field must be true.
- Do not output markdown fencing or commentary.
</rules>

<project_description>{PROJECT_DESCRIPTION}</project_description>"#;

/// Prompt for selecting relevant public API endpoints for a project.
pub(crate) const ARCHITECT_ENDPOINTS_PROMPT: &str = r#"<role>You are a master software architect. Identify public API endpoints that align precisely with the project behavior.</role>

<rules>
- Prioritize stable endpoints that do not require API keys.
- Return ONLY a valid JSON array of URL strings: `["url_1", "url_2"]`
- Do not output markdown fencing or commentary.
</rules>

<project_description>{PROJECT_DESCRIPTION}</project_description>"#;

/// Prompt for generating Python architecture diagrams using the diagrams library.
pub(crate) const ARCHITECT_DIAGRAM_PROMPT: &str = r#"<role>You are a software architect plotting cloud infrastructures. Generate Python scripts using the `diagrams` library to construct the project architecture diagram.</role>

<rules>
- Output ONLY the raw Python source code.
- Provide no backticks, no markdown fencing, no commentary,.
- Your code must be runnable without syntax errors.
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
