package com.gemwallet.android.ext

import com.gemwallet.android.domains.name.AddressInputResolving
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientServiceInterface
import uniffi.gemstone.GemRecipientValidation

fun GemNameServiceInterface.addressInput(): AddressInputResolving =
    ServiceAddressInput(::validateRecipient, ::recipient, ::isNameSupported, ::getNameRecord)

fun GemRecipientServiceInterface.addressInput(): AddressInputResolving =
    ServiceAddressInput(::validateRecipient, ::recipient, ::isNameSupported, ::getNameRecord)

private class ServiceAddressInput(
    private val validate: (String, String, String?) -> GemRecipientValidation,
    private val build: (String, String, String?, String?, List<String>) -> GemRecipient,
    private val supported: (String) -> Boolean,
    private val record: suspend (String, String) -> String?,
) : AddressInputResolving {

    override fun validateRecipient(chain: Chain, input: String, nameRecord: NameRecord?): GemRecipientValidation =
        validate(chain.string, input, nameRecord?.toJson())

    override fun recipient(chain: Chain, input: String, nameRecord: NameRecord?, memo: String?, references: List<String>): GemRecipient =
        build(chain.string, input, nameRecord?.toJson(), memo, references)

    override fun isNameSupported(name: String): Boolean = supported(name)

    override suspend fun getNameRecord(name: String, chain: Chain): NameRecord? = withContext(Dispatchers.IO) {
        record(name, chain.string)?.decodeJson<NameRecord>()
    }
}
