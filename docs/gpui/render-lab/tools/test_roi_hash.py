import unittest

from PIL import Image

from roi_hash import find_marker_bbox


class FiducialTests(unittest.TestCase):
    def test_closed_marker_returns_inner_roi(self):
        image = Image.new("RGBA", (431, 91), (0, 0, 0, 255))
        pixels = image.load()
        for x in range(431):
            pixels[x, 0] = pixels[x, 90] = (255, 0, 255, 255)
        for y in range(91):
            pixels[0, y] = pixels[430, y] = (255, 0, 255, 255)
        self.assertEqual(find_marker_bbox(image), (2, 2, 427, 87))

    def test_truncated_marker_is_rejected(self):
        image = Image.new("RGBA", (431, 91), (0, 0, 0, 255))
        pixels = image.load()
        for x in range(431):
            pixels[x, 0] = pixels[x, 90] = (255, 0, 255, 255)
        for y in range(91):
            pixels[0, y] = (255, 0, 255, 255)
        with self.assertRaises(SystemExit) as error:
            find_marker_bbox(image)
        self.assertIn("truncated", str(error.exception))


if __name__ == "__main__":
    unittest.main()
