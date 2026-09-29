// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing

struct FormatterCacheTests {
    @Test
    func reusesTheFormatterForTheSameKey() {
        let first = FormatterCache.formatter(key: "FormatterCacheTests|same") { DateFormatter() }
        let second = FormatterCache.formatter(key: "FormatterCacheTests|same") { DateFormatter() }
        #expect(first === second)
    }

    @Test
    func buildsOneFormatterPerKey() {
        let short = FormatterCache.formatter(key: "FormatterCacheTests|short") { DateFormatter() }
        let long = FormatterCache.formatter(key: "FormatterCacheTests|long") { DateFormatter() }
        #expect(short !== long)
    }
}
