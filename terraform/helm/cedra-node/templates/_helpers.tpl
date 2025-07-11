{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "cedra-validator.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "cedra-validator.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "cedra-validator.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Common labels
*/}}
{{- define "cedra-validator.labels" -}}
{{- range $k, $v := .Values.labels }}
{{ $k }}: {{ $v }}
{{- end }}
helm.sh/chart: {{ include "cedra-validator.chart" . }}
{{ include "cedra-validator.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
Multicluster labels. `multiclusterLabels` takes in a tuple of context and index as arguments.
It should be invoked as `cedra-validator.multiclusterLabels (tuple $ $i)` where $i is the index
of the statefulset.

The logic below assigns a target cluster to each statefulset replica in a round-robin fashion.
*/}}
{{- define "cedra-validator.multiclusterLabels" -}}
{{- $ctx := index $ 0 -}}
{{- if $ctx.Values.multicluster.enabled }}
{{- $index := index $ 1 -}}
{{- $numClusters := len $ctx.Values.multicluster.targetClusters }}
{{- $clusterIndex := mod $index $numClusters }}
{{- $cluster := index $ctx.Values.multicluster.targetClusters $clusterIndex }}
multicluster/targetcluster: {{ $cluster }}
{{- end }}
{{- end -}}

{{/*
Selector labels
*/}}
{{- define "cedra-validator.selectorLabels" -}}
{{- range $k, $v := .Values.labels }}
{{ $k }}: {{ $v }}
{{- end }}
app.kubernetes.io/part-of: {{ include "cedra-validator.name" . }}
app.kubernetes.io/managed-by: helm
{{- end -}}

{{/*
Create the name of the service account to use
*/}}
{{- define "cedra-validator.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
    {{ default (include "cedra-validator.fullname" .) .Values.serviceAccount.name }}
{{- else -}}
    {{ default "default" .Values.serviceAccount.name }}
{{- end -}}
{{- end -}}
