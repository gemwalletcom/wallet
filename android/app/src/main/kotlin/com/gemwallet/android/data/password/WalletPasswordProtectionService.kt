package com.gemwallet.android.data.password

import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import kotlinx.coroutines.flow.first
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class WalletPasswordProtectionService @Inject constructor(private val passwords: TinkPasswordStore, private val walletsQuery: WalletsQuery) : WalletPasswordProtection {
    override fun authenticationRequired(): Boolean = passwords.authenticationRequired()

    override suspend fun setAuthenticationRequired(required: Boolean) {
        passwords.setAuthenticationRequired(required, walletsQuery().first().map { it.id.id })
    }
}
