package com.gemwallet.android.features.contacts.viewmodels

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemChainRow
import uniffi.gemstone.GemChainServiceInterface
import javax.inject.Inject

@HiltViewModel
class ContactChainSelectViewModel @Inject constructor(private val chainService: GemChainServiceInterface) : ViewModel() {
    fun chains(query: String): List<GemChainRow> = chainService.chainRows(null, query)
}
