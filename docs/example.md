# Example API Requests

## AlibabaCloud

```bash
curl -X POST https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions \
-H "Authorization: Bearer $DASHSCOPE_API_KEY" \
-H "Content-Type: application/json" \
-d '{
    "model": "qwen-plus",
    "messages": [
        {
            "role": "system",
            "content": "You are a helpful assistant."
        },
        {
            "role": "user",
            "content": "Who are you?"
        }
    ]
}'
```

## DeepInfra

```bash
curl "https://api.deepinfra.com/v1/openai/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_DEEPINFRA_API_KEY" \
  -d '{
      "model": "Qwen/Qwen3.5-2B",
      "messages": [
        {
          "role": "user",
          "content": "Hello!"
        }
      ]
    }'
```

## Gemini

```bash
curl https://generativelanguage.googleapis.com/v1beta/openai/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_GOOGLE_AI_STUDIO_KEY" \
  -d '{
    "model": "gemini-3-flash-preview",
    "messages": [
      {
        "role": "system",
        "content": "You are a poet."
      },
      {
        "role": "user",
        "content": "Write a two-line poem about coffee."
      }
    ]
  }'
```
