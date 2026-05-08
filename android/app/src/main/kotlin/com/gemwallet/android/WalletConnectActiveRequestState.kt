package com.gemwallet.android

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class WalletConnectActiveRequestState @Inject constructor() {

    private val _hasActive = MutableStateFlow(false)
    val hasActive: StateFlow<Boolean> = _hasActive.asStateFlow()

    fun setActive(active: Boolean) {
        _hasActive.value = active
    }
}
