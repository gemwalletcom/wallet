package com.gemwallet.android.features.support.viewmodels

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.device.cases.EnablePushForSupport
import com.gemwallet.android.application.support.cases.ClearSupportTyping
import com.gemwallet.android.application.support.cases.GetSupportTyping
import com.gemwallet.android.data.services.store.queries.SupportMessagesQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPushResult
import uniffi.gemstone.GemSupportServiceInterface
import uniffi.gemstone.loadError
import javax.inject.Inject

@HiltViewModel
class SupportChatViewModel @Inject constructor(
    private val supportService: GemSupportServiceInterface,
    private val supportMessagesQuery: SupportMessagesQuery,
    private val getSupportTyping: GetSupportTyping,
    private val clearSupportTyping: ClearSupportTyping,
    private val enablePushForSupport: EnablePushForSupport,
    private val imageAttachmentFactory: SupportImageAttachmentFactory,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val messages = supportMessagesQuery()
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

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val errorRow: StateFlow<GemListRow?> = combine(loadState, messages) { state, shown ->
        loadError(state, shown.isNotEmpty())?.let { GemListRow.Error(it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    init {
        viewModelScope.launch(ioDispatcher) {
            val state = enablePushForSupport.enablePushForSupport() ?: return@launch
            (state.result as? GemPushResult.NotRegistered)?.let { errorState.value = it.error.text(context) }
        }
    }

    fun fetch() = viewModelScope.launch(ioDispatcher) {
        val shown = messages.first()
        val fromTimestamp = supportService.syncFromTimestamp(shown.map { it.toGem() })
        loadState.update { supportService.refresh(fromTimestamp, shown.isNotEmpty()) }
    }

    fun sendText(content: String) = viewModelScope.launch(ioDispatcher) {
        alertOnFailure { supportService.sendText(content) }
    }

    fun sendImages(uris: List<Uri>) = viewModelScope.launch(ioDispatcher) {
        for (uri in uris) {
            alertOnFailure {
                val image = imageAttachmentFactory.fromUri(uri)
                if (image == null) {
                    errorState.value = GemErrorText.NotSupported.text(context)
                    return@alertOnFailure
                }
                supportService.sendImage(image)
            }
        }
    }

    fun retry(message: SupportMessage) = viewModelScope.launch(ioDispatcher) {
        alertOnFailure { supportService.retryMessage(message.toGem()) }
    }

    override fun onCleared() {
        super.onCleared()
        clearSupportTyping.clearTyping()
    }

    private suspend fun alertOnFailure(block: suspend () -> Unit) {
        runCatchingCancellable(block).onFailure { errorState.value = it.errorText().text(context) }
    }

    fun clearError() = errorState.update { null }

    companion object {
        private const val TAG = "SupportChat"
    }
}
