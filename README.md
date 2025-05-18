# Local Dev Stack

Spin up the full backend shared platform locally.

## Included Services

- Redis (cache and stream)
- MySQL (service DB)
- Keycloak (auth)
- Unleash (feature toggles)
- ElasticSearch + Kibana (logs)
- Prometheus + Grafana (metrics)

## Dependencies

brew install dnsmasq

## Start docker

```bash
docker compose -f docker-compose.local.yml up --build
```

## Start k8

	•	http://unleash.127.0.0.1.nip.io
	•	http://keycloak.127.0.0.1.nip.io
	•	http://grafana.127.0.0.1.nip.io
	•	http://prometheus.127.0.0.1.nip.io
	•	http://kibana.127.0.0.1.nip.io
	•	http://traefik.127.0.0.1.nip.io
