from langchain_community.chat_models import ChatOpenAI
from langchain.schema import HumanMessage

# Bind to HMIR exactly exposing native components directly dynamically mapping OpenAI streams securely!
chat = ChatOpenAI(
    openai_api_key="hmir-local",
    openai_api_base="http://localhost:8080/v1",
    model_name="llama3-8b-hmir-optimized",
    streaming=True
)

response = chat([HumanMessage(content="Explain HMIR memory caching mapping!")])
print(response.content)
