package com.gemwallet.android.domains.balance

const val HIDDEN_BALANCE = "*****"

fun String.hiddenWhen(hidden: Boolean): String = if (hidden) HIDDEN_BALANCE else this
