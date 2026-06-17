package com.gemwallet.android.features.settings.settings.viewmodels

import android.net.Uri
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.ext.millisToSeconds
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class SupportChatSceneViewModel @Inject constructor(
    private val repository: SupportChatRepository,
    private val imageAttachmentFactory: SupportImageAttachmentFactory,
) : ViewModel() {

    private val messages = repository.getMessages()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val days = messages
        .map(::buildSupportChatDays)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val isEmpty = messages
        .map { it.isEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val typingAgentName = repository.typing
        .map { it?.name }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun fetch() = viewModelScope.launch(Dispatchers.IO) {
        perform("fetch") {
            repository.failPendingMessages()
            val fromTimestamp = repository.getMessages().first()
                .lastOrNull { it.sender is SupportMessageSender.Agent }
                ?.let { it.createdAt.millisToSeconds() } ?: 0L
            repository.syncMessages(fromTimestamp)
        }
    }

    fun sendText(content: String) = viewModelScope.launch(Dispatchers.IO) {
        perform("send text") { repository.sendText(content) }
    }

    fun sendImage(uri: Uri) = viewModelScope.launch(Dispatchers.IO) {
        val attachment = imageAttachmentFactory.fromUri(uri) ?: return@launch
        perform("send image") { repository.sendImage(attachment) }
    }

    fun retry(message: SupportMessage) {
        if (message.images.isNotEmpty()) return
        viewModelScope.launch(Dispatchers.IO) {
            perform("retry") { repository.retryMessage(message) }
        }
    }

    override fun onCleared() {
        super.onCleared()
        repository.clearTyping()
    }

    private suspend fun perform(context: String, block: suspend () -> Unit) {
        try {
            block()
        } catch (err: Throwable) {
            Log.e(TAG, "$context error", err)
        }
    }

    companion object {
        private const val TAG = "SupportChat"
    }
}
