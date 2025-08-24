Membuat web service dengan rust

cara penggunaan:
1 Login:
	curl -X POST http://localhost:8080/api/login -H "Content-Type: application/json" -d '{"username":"admin","password":"gedanggoreng"}'

2 Get User Info:
	curl -X GET http://localhost:8080/api/user -H "Authorization: Bearer <token_yang_didapat_dari_login>"
3 Logout:
 	curl -X POST http://localhost:8080/api/logout -H "Authorization: Bearer <token_yang_didapat_dari_login>"

