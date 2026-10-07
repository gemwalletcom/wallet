package com.gemwallet.android.features.contacts.viewmodels

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemChainList
import uniffi.gemstone.GemChainServiceInterface
import javax.inject.Inject

@HiltViewModel
class ContactChainSelectViewModel @Inject constructor(private val chainService: GemChainServiceInterface) : ViewModel() {
    fun chains(query: String): GemChainList = chainService.chainList(null, query)
}
