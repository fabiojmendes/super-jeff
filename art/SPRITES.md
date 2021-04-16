## Using Krita and ImageMagick to create sprites

1. Render the animation frames in Krita
2. Use the convert command to append images together

```
convert +append ./sequence* sprite.png
```

* Use `-append` for vertical and `+append` for horizontal
