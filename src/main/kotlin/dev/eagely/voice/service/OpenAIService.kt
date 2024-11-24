package dev.eagely.voice.service

import com.aallam.openai.api.chat.ChatCompletion
import com.aallam.openai.api.chat.ChatCompletionRequest
import com.aallam.openai.api.chat.ChatMessage
import com.aallam.openai.api.chat.ChatRole
import com.aallam.openai.api.model.ModelId
import com.aallam.openai.client.OpenAI
import kotlinx.coroutines.runBlocking
import org.springframework.stereotype.Service
import reactor.core.publisher.Mono

@Service
class OpenAIService {
    private val apiKey = System.getenv("OPENAI_API_KEY")
    private val openAI = OpenAI(apiKey)

    fun getCompletion(content: String): Mono<ChatCompletion> {
        val chatCompletionRequest = ChatCompletionRequest(
            model = ModelId("gpt-4o-mini"), messages = listOf(
                ChatMessage(
                    role = ChatRole.System,
                    content = "You are a helpful voice assistant, reply naturally, without describing what task you will perform."
                ), ChatMessage(
                    role = ChatRole.User, content = content
                )
            )
        )
        return Mono.fromCallable { runBlocking { openAI.chatCompletion(chatCompletionRequest) } }
    }
}