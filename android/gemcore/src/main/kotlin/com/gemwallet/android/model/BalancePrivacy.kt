package com.gemwallet.android.model

const val HIDDEN_BALANCE_MASK = "✱✱✱✱✱"

fun String.masked(hideBalance: Boolean): String = if (hideBalance) HIDDEN_BALANCE_MASK else this

fun String?.maskedOrNull(hideBalance: Boolean): String? = this?.masked(hideBalance)
