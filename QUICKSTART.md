# Quickstart

## Cards
Cards have four features: *Shape*, *Count*, *Color*, and *Shading*. Each feature has three possible values:

> Shape: Oval, Diamond, Squiggle
>
> Count: 1, 2, 3
>
> Color: Cyan, Magenta, Yellow (or Red, Green, Purple when using classic colors)
>
> Shading: Solid, Outlined, Striped (drawn in Marmoset as a translucenct fill with a white border)

If you are a beginner, you might want to select **Game > Deck > Beginner** to use a deck that contains only the Solid cards.

Select and deselect cards by clicking on them or by typing the hotkey that appears in the bottom-left corner of the card.

## Playing Set

The goal is to select 3 cards that form a **Set**. A Set is formed when each feature is either the same across all three cards, or different for each card.

For example, in the Set below, everything is the same except the Count:

![Marmoset Card](/images/54.png)
![Marmoset Card](/images/27.png)
![Marmoset Card](/images/0.png)

In this Set, the Count and the Shading are the same, but the Color and Shape differ:

![Marmoset Card](/images/28.png)
![Marmoset Card](/images/40.png)
![Marmoset Card](/images/52.png)

And in this Set, all four features are different on each card:

![Marmoset Card](/images/54.png)
![Marmoset Card](/images/41.png)
![Marmoset Card](/images/25.png)

When you have successfully selected a Set, the cards will be removed and replaced with new cards from the stock. The game ends when the stock is empty and no more Sets are in play.

If you get stuck, select **Control > Hint** for a hint. Marmoset will preselect 2 cards that are part of an available Set (or deal more cards if no Sets are available).

If you think there are no Sets in play, select **Control > Deal More Cards** to add more cards.

### Playing SuperSet

The goal is to select 4 cards that form a **[SuperSet]**.

Given any 2 cards, there is exactly 1 card that will form a Set with those cards. The 4 cards in a SuperSet can be split into 2 pairs that each share a common Set-completing card. For example, consider the SuperSet below:

![Marmoset Card](/images/54.png)
![Marmoset Card](/images/57.png)
![Marmoset Card](/images/51.png)
![Marmoset Card](/images/15.png)

In this case, the first pair is:

![Marmoset Card](/images/54.png)
![Marmoset Card](/images/57.png)

And the second pair is:

![Marmoset Card](/images/51.png)
![Marmoset Card](/images/15.png)

In both pairs, the card needed to complete the Set is:

![Marmoset Card](/images/60.png)

It's important to note that the fifth card that is shared between the pairs is implied and does not need to be in play.

When you have successfully selected a SuperSet, the cards will be removed and replaced with new cards from the stock. The game ends when the stock is empty and no more SuperSets are in play.

There will always be SuperSets available as long as 10 cards are on the board. If you get stuck, **Control > Hint** will preselect 2 cards that form one of the pairs in an available SuperSet. Figure out the third card that completes the Set with that pair and you can isolate the remaining 2 cards.

See the original SuperSet description [here](http://magliery.com/Set/SuperSet.html) for more details.

[SuperSet]: http://magliery.com/Set/SuperSet.html
