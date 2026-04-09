package com.gemwallet.android.serializer

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.Serializer
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import java.time.Instant
import java.time.ZoneOffset
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter

@OptIn(ExperimentalSerializationApi::class)
@Serializer(forClass = Long::class)
object DateSerializer {

    override fun deserialize(decoder: Decoder): Long {
        return try {
            ZonedDateTime.parse(decoder.decodeString(), DateTimeFormatter.ISO_ZONED_DATE_TIME)
                .toInstant()
                .toEpochMilli()
        } catch (_: Throwable) {
            0
        }
    }

    override fun serialize(encoder: Encoder, value: Long) {
        val isoTimestamp = ZonedDateTime.ofInstant(
            Instant.ofEpochMilli(value),
            ZoneOffset.UTC
        )
            .format(DateTimeFormatter.ISO_ZONED_DATE_TIME)
        encoder.encodeString(isoTimestamp)
    }
}
