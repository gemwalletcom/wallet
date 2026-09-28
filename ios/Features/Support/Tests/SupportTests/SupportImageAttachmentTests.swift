// Copyright (c). Gem Wallet. All rights reserved.

@testable import Support
import Testing
import UIKit

struct SupportImageAttachmentTests {
    @Test
    func aLargePhotoIsScaledToTheLimitAndASmallOneIsKept() {
        let format = UIGraphicsImageRendererFormat.default()
        format.scale = 1
        let large = UIGraphicsImageRenderer(size: CGSize(width: 4000, height: 3000), format: format).image { _ in }
        let small = UIGraphicsImageRenderer(size: CGSize(width: 800, height: 600), format: format).image { _ in }

        #expect(large.fitting(maxDimension: 2048).size == CGSize(width: 2048, height: 1536))
        #expect(small.fitting(maxDimension: 2048).size == CGSize(width: 800, height: 600))
    }
}
