# actix_api_example

Login:
	curl -X POST http://localhost:8080/api/login -H "Content-Type: application/json" -d '{"username":"admin","password":"gedanggoreng"}'

Get User Info:
	curl -X GET http://localhost:8080/api/user -H "Authorization: Bearer <token_yang_didapat_dari_login>"

 Logout:
 	curl -X POST http://localhost:8080/api/logout -H "Authorization: Bearer <token_yang_didapat_dari_login>"
