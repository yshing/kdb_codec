assertEq:{[name; sent; recv]
  if[not sent~recv;
    -2 "assert failed: ", name;
    -2 "  sent: ", -3!sent;
    -2 "  recv: ", -3!recv;
    '"assertion failed";
  ];
  ::
 };
