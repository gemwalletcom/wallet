package com.gemwallet.android.features.settings.settings.viewmodels

import android.net.Uri
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.FailPendingSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportMessages
import com.gemwallet.android.application.support.cases.GetSupportTyping
import com.gemwallet.android.ext.millisToSeconds
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import uniffi.gemstone.GemSupportServiceInterface
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
    private val supportService: GemSupportServiceInterface,
    private val getSupportMessages: GetSupportMessages,
    private val failPendingSupportMessages: FailPendingSupportMessages,
    private val getSupportTyping: GetSupportTyping,
    private val clearSupportTyping: ClearSupportTyping,
    private val imageAttachmentFactory: SupportImageAttachmentFactory,
) : ViewModel() {

    private val messages = getSupportMessages()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val days = messages
        .map(::buildSupportChatDays)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val isEmpty = messages
        .map { it.isEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val typingAgentName = getSupportTyping.typingAgent()
        .map { it?.name }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun fetch() = viewModelScope.launch(Dispatchers.IO) {
        perform("fetch") {
            failPendingSupportMessages()
            val fromTimestamp = messages.first()
                .lastOrNull { it.sender is SupportMessageSender.Agent }
                ?.let { it.createdAt.millisToSeconds() } ?: 0L
            supportService.syncMessages(fromTimestamp.toULong())
        }
    }

    fun sendText(content: String) = viewModelScope.launch(Dispatchers.IO) {
        perform("send text") { supportService.sendText(content) }
    }

    fun sendImage(uri: Uri) = viewModelScope.launch(Dispatchers.IO) {
        val attachment = imageAttachmentFactory.fromUri(uri) ?: return@launch
        perform("send image") { supportService.sendImage(attachment.data, attachment.fileName, attachment.mimeType) }
    }

    fun retry(message: SupportMessage) {
        if (message.images.isNotEmpty()) return
        viewModelScope.launch(Dispatchers.IO) {
            perform("retry") { supportService.retryMessage(message.toJson()) }
        }
    }

    override fun onCleared() {
        super.onCleared()
        clearSupportTyping.clearTyping()
    }

    private suspend fun perform(context: String, block: suspend () -> Unit) {
        runCatchingCancellable(block).onFailure { Log.e(TAG, "$context error", it) }
    }

    companion object {
        private const val TAG = "SupportChat"
    }
}
