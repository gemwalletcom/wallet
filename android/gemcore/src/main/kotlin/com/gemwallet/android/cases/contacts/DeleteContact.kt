package com.gemwallet.android.cases.contacts

interface DeleteContact {
    suspend fun deleteContact(id: String)
}
