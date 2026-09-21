package com.gemwallet.android.features.settings.networks.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import uniffi.gemstone.GemServiceStatusInterface
import javax.inject.Inject

@HiltViewModel
class ServiceStatusViewModel @Inject constructor(private val service: GemServiceStatusInterface) : ViewModel() {
    private val _sections = MutableStateFlow(service.sections())
    val sections = _sections.asStateFlow()
    private var fetchJob: Job? = null

    fun fetch() {
        fetchJob?.cancel()
        _sections.value = service.sections()
        fetchJob = viewModelScope.launch {
            val result = service.load()
            ensureActive()
            _sections.value = result
        }
    }
}
