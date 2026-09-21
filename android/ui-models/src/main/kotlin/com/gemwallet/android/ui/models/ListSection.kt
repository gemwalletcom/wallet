package com.gemwallet.android.ui.models

data class ListSection<T>(val id: String, val title: String? = null, val items: List<T>, val footer: String? = null)
