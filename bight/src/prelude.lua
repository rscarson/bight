function POSX()
  x, _ = POS()
  return x
end

function POSY()
  _, y = POS()
  return y
end

function REL(deltaX, deltaY)
  x, y = POS()
  GET(x + deltaX, y + deltaY)
end

setmetatable(_G, {
  __index = function(_, index)
    return GET(index)
  end
})
