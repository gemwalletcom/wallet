// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Testing

struct ConnectionComponentHealthTests {
    @Test
    func healthStreamReplaysTheLastReport() async {
        let health = ConnectionComponentHealth(component: .stream)
        health.report(isHealthy: true)

        var values = health.healthStream().makeAsyncIterator()

        #expect(await values.next() == true)
    }

    @Test
    func everyStreamReceivesTheNextReport() async {
        let health = ConnectionComponentHealth(component: .stream)
        var first = health.healthStream().makeAsyncIterator()
        var second = health.healthStream().makeAsyncIterator()

        health.report(isHealthy: false)

        #expect(await first.next() == false)
        #expect(await second.next() == false)
    }
}
