import func Gemstone.addressCopy
import func Gemstone.privateKeyCopy
import func Gemstone.secretPhraseCopy
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing
import UIKit

struct ClipboardTests {
    @Test
    func pasteboardOptionsNil() {
        let options = Clipboard.pasteboardOptions(expirationTime: nil)

        #expect(options.isEmpty)
    }

    @Test
    func pasteboardOptionsWithExpiration() {
        let options = Clipboard.pasteboardOptions(expirationTime: 60)

        #expect(options[.localOnly] as? Bool == true)
        #expect(options[.expirationDate] as? Date != nil)
    }

    @Test
    func aSecretExpiresAndAnAddressStays() {
        #expect(Clipboard.expirationTime(secretPhraseCopy(words: [])) == 60)
        #expect(Clipboard.expirationTime(privateKeyCopy(key: "")) == 60)
        #expect(Clipboard.expirationTime(addressCopy(chain: Chain.ethereum.toGem(), address: "")) == nil)
    }
}
