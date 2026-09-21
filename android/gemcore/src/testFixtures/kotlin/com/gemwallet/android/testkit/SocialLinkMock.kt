package com.gemwallet.android.testkit

import uniffi.gemstone.GemSocialLink
import uniffi.gemstone.LinkType

fun mockGemSocialLink(linkType: LinkType = LinkType.WEBSITE, url: String = "https://bitcoin.org", host: String? = "bitcoin.org") = GemSocialLink(linkType = linkType, url = url, host = host)
